//! Native67-byte building storage with explicit animation bindings.
//! Admission takes native priors; packed59-byte loader conversion is a separate boundary.
use crate::building::{BuildingRecord,BuildingAnimationBinding,BuildingFactoryPlan};
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum BuildingPoolError{PackedLength(usize),ActivePackedRecord(usize),InvalidSlot,InvalidRecord(usize),InvalidBinding(usize),StaleRecord,InvalidTransition}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct BuildingPool{records:[BuildingRecord;256],bindings:[Option<BuildingAnimationBinding>;256]}
fn validate(slot:usize,record:&BuildingRecord,binding:Option<BuildingAnimationBinding>)->Result<(),BuildingPoolError>{
    if record[6]==0 {return if binding.is_none(){Ok(())}else{Err(BuildingPoolError::InvalidBinding(slot))};}
    if record[6]!=4||i16::from_le_bytes(record[15..17].try_into().unwrap())!=slot as i16
        ||usize::from(record[24])!=slot/64||record[45]>=14{return Err(BuildingPoolError::InvalidRecord(slot));}
    if binding!=Some(BuildingAnimationBinding{selector:record[45]}){return Err(BuildingPoolError::InvalidBinding(slot));}
    Ok(())
}
impl BuildingPool{
    pub fn admit(records:[BuildingRecord;256],bindings:[Option<BuildingAnimationBinding>;256])->Result<Self,BuildingPoolError>{
        for i in 0..256{validate(i,&records[i],bindings[i])?;}Ok(Self{records,bindings})
    }
    pub fn records(&self)->&[BuildingRecord;256]{&self.records}
    pub fn record(&self,slot:usize)->Result<&BuildingRecord,BuildingPoolError>{self.records.get(slot).ok_or(BuildingPoolError::InvalidSlot)}
    pub fn binding(&self,slot:usize)->Result<Option<BuildingAnimationBinding>,BuildingPoolError>{self.bindings.get(slot).copied().ok_or(BuildingPoolError::InvalidSlot)}
    pub fn apply_factory(&mut self,plan:&BuildingFactoryPlan)->Result<(),BuildingPoolError>{
        let slot=usize::from(plan.slot);let prior=self.record(slot)?;
        if prior!=&plan.before{return Err(BuildingPoolError::StaleRecord);}
        if prior[6]!=0{return Err(BuildingPoolError::InvalidTransition);}
        validate(slot,&plan.after,Some(plan.binding))?;
        self.records[slot]=plan.after;self.bindings[slot]=Some(plan.binding);Ok(())
    }
    pub fn replace_record_checked(&mut self,slot:usize,before:&BuildingRecord,after:&BuildingRecord)->Result<(),BuildingPoolError>{
        if self.record(slot)?!=before{return Err(BuildingPoolError::StaleRecord);}
        let binding=if after[6]==0{None}else{self.bindings[slot]};validate(slot,after,binding)?;
        self.records[slot]=*after;self.bindings[slot]=binding;Ok(())
    }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn inactive_priors_are_preserved_and_binding_mismatch_rejects(){
        let mut records=[[0xa5;67];256];for r in &mut records{r[6]=0;}
        let pool=BuildingPool::admit(records,[None;256]).unwrap();assert_eq!(pool.records(),&records);
        let mut bindings=[None;256];bindings[0]=Some(BuildingAnimationBinding{selector:1});assert_eq!(BuildingPool::admit(records,bindings),Err(BuildingPoolError::InvalidBinding(0)));
    }
}

impl BuildingPool {
    pub fn from_packed_inactive(
        packed: &[u8],
        prior: &[BuildingRecord; 256],
        bindings: [Option<BuildingAnimationBinding>; 256],
    ) -> Result<Self, BuildingPoolError> {
        if packed.len() != 256 * 59 {
            return Err(BuildingPoolError::PackedLength(packed.len()));
        }

        let mut records = *prior;
        for (slot, source) in packed.chunks_exact(59).enumerate() {
            // Source 45010 calls a virtual initializer for nonzero +6. This
            // API admits only the source-inactive path and rejects beforehand.
            if source[6] != 0 {
                return Err(BuildingPoolError::ActivePackedRecord(slot));
            }
            if bindings[slot].is_some() {
                return Err(BuildingPoolError::InvalidBinding(slot));
            }

            // 6D490 copies 31 bytes; 45010 copies the remaining 28 bytes.
            // The eight-byte native-only gap is preserved from `prior`.
            records[slot][..31].copy_from_slice(&source[..31]);
            records[slot][39..67].copy_from_slice(&source[31..59]);
            validate(slot, &records[slot], bindings[slot])?;
        }
        Ok(Self { records, bindings })
    }
}

impl BuildingPool {pub(crate) fn fixture_set(&mut self,i:usize,r:&[u8]){self.records[i].copy_from_slice(r);self.bindings[i]=Some(crate::building::BuildingAnimationBinding{selector:r[45]});}}
